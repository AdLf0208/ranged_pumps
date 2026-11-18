//! Ranged Pumps — the pump block entity, its scan and its tank, as one `impl Hooks`.
//!
//! `src/runtime.rs` is the port's pure logic (the scan, the still-source rule, the tank and its
//! state) and carries its own tests; this file is only the game glue.
//!
//! Upstream ticks a pump's block entity: every configured `speed` ticks it moves one block down
//! the current column (then the next column of the ring), drains a still source block into its
//! one 32-bucket tank, replaces the source with stone, and each tick pushes its tank into
//! neighbouring fluid handlers. The tank is the game's default fluid tanks for a mod block
//! (`Registrar::block_fluids`), saved in the entity's `fluids` compound; neighbours are reached
//! through the `horizon:fluids` capability and filled with `ResourceHandler::insert_any`.
mod runtime;

use mod_api::{
    self, BlockEntityPart, BlockEntityRef, BlockEntityTickEvent, BlockEntityTicksEvent, BlockPart,
    BlockUpdateFlags, Capability, Command, CommandArgumentValue, Compound, Direction, FLUIDS, Flow,
    FluidResource, FluidStack, Hooks, HostTransaction, InteractionResult, LevelRef, ModFluidTanks,
    RegisterCommandsEvent, Registrar, RegistrationContext, ResourceHandler, RunModCommandEvent,
    ServerHost, Tag, Text, UseWithoutItemEvent,
};

use runtime::{
    DEFAULT_RANGE, MAX_RANGE, Pos, PumpState, SPEED_TICKS, Scan, TANK_CAPACITY, Tank, source_fluid,
};

/// The block and its item.
const PUMP: &str = "rangedpumps:pump";
/// What a drained source is replaced with (upstream `blockIdToReplaceLiquidsWith`).
const REPLACE_WITH: &str = "minecraft:stone";

// The pump's saved scan and tank, in the shapes the game keeps a mod block entity's compound and
// its declared tanks in: `range`/`columnIndex`/`currentPos` are upstream's three fields, and the
// tank list is `fluids` (`Fluid` + `Amount`), written by the game's default fluid provider.
//
// A compound's key type is UTF-16 and the API does not export it, so a key is taken from
// `Tag::String`'s own payload: the same conversion a mod's NBT string uses, with no name needed.

fn key_tag(name: &str) -> Tag {
    Tag::String(name.into())
}

fn tag<'a>(nbt: &'a Compound, name: &str) -> Option<&'a Tag> {
    let Tag::String(utf16) = key_tag(name) else {
        return None;
    };
    nbt.get(&utf16)
}

fn put(nbt: &mut Compound, name: &str, value: Tag) {
    if let Tag::String(utf16) = key_tag(name) {
        nbt.insert(utf16, value);
    }
}

fn drop_tag(nbt: &mut Compound, name: &str) {
    if let Tag::String(utf16) = key_tag(name) {
        nbt.remove(&utf16);
    }
}

fn int(nbt: &Compound, name: &str) -> Option<i32> {
    match tag(nbt, name) {
        Some(Tag::Int(value)) => Some(*value),
        _ => None,
    }
}

fn long(nbt: &Compound, name: &str) -> Option<i64> {
    match tag(nbt, name) {
        Some(Tag::Long(value)) => Some(*value),
        _ => None,
    }
}

/// Java `BlockPos.asLong`, the shape upstream saves `currentPos` in.
fn pack_pos([x, y, z]: Pos) -> i64 {
    ((i64::from(x) & 0x3FF_FFFF) << 38)
        | ((i64::from(z) & 0x3FF_FFFF) << 12)
        | (i64::from(y) & 0xFFF)
}

/// Java `BlockPos.getX/getY/getZ` (sign-extending each field).
fn unpack_pos(value: i64) -> Pos {
    [
        (value >> 38) as i32,
        ((value << 52) >> 52) as i32,
        ((value << 26) >> 38) as i32,
    ]
}

fn read_scan(nbt: &Compound) -> Scan {
    Scan::from_saved(
        int(nbt, "range").unwrap_or(-1),
        int(nbt, "columnIndex").unwrap_or(0),
        long(nbt, "currentPos").map(unpack_pos),
    )
}

fn write_scan(nbt: &mut Compound, scan: &Scan) {
    let (range, column_index, current) = scan.saved();
    put(nbt, "range", Tag::Int(range));
    put(nbt, "columnIndex", Tag::Int(column_index));
    match current {
        Some(pos) => {
            put(nbt, "currentPos", Tag::Long(pack_pos(pos)));
        }
        None => {
            drop_tag(nbt, "currentPos");
        }
    }
}

/// The pump's configured ring radius (upstream `ServerConfig.range`); this port keeps it per
/// block so a pump's range can be set and read in game.
fn configured_range(nbt: &Compound) -> i32 {
    int(nbt, "configuredRange")
        .unwrap_or(DEFAULT_RANGE)
        .clamp(0, MAX_RANGE)
}

/// The entity's first tank, as the game keeps it under `fluids`.
fn read_tank(nbt: &Compound) -> Tank {
    let Some(Tag::List(list)) = tag(nbt, "fluids") else {
        return Tank::EMPTY;
    };
    let Some(Tag::Compound(entry)) = list.first() else {
        return Tank::EMPTY;
    };
    let fluid = match tag(entry, "Fluid") {
        Some(Tag::String(fluid)) if fluid.to_string_lossy() != "minecraft:empty" => {
            fluid.to_string_lossy()
        }
        _ => return Tank::EMPTY,
    };
    let amount = match tag(entry, "Amount") {
        Some(Tag::Int(amount)) => *amount,
        _ => 0,
    };
    Tank::new(&fluid, amount)
}

/// Write the first tank back in the game's `fluids` shape.
fn write_tank(nbt: &mut Compound, tank: &Tank) {
    let mut entry = Compound::new();
    put(
        &mut entry,
        "Fluid",
        Tag::String(tank.fluid.as_deref().unwrap_or("minecraft:empty").into()),
    );
    put(&mut entry, "Amount", Tag::Int(tank.amount));
    put(nbt, "fluids", Tag::List(vec![Tag::Compound(entry)]));
}

/// The authoritative level the server ticks now: the one a block entity's tick belongs to (a
/// parked level's block entities are not ticked, and a client of a server ticks no level).
fn in_place_level(host: &mut dyn ServerHost) -> Option<LevelRef> {
    host.levels()
        .list_levels()
        .into_iter()
        .find(|level| level.in_place(host))
}

/// The pump block entity at `pos`, if the block there is ours.
fn pump_at(host: &mut dyn ServerHost, level: LevelRef, pos: Pos) -> Option<BlockEntityRef> {
    let block = level.block(host, pos)?;
    (block.id(host) == PUMP)
        .then(|| host.block_entity(level, pos))
        .flatten()
}

/// The state and tank a pump reports, as upstream's `PumpState` line and tank line.
fn readout(nbt: &Compound, pos: Pos) -> (String, String) {
    let scan = read_scan(nbt);
    let max_range = configured_range(nbt);
    let tank = read_tank(nbt);
    let state_line = match PumpState::of(&scan, max_range, &tank) {
        PumpState::Working => {
            let at = scan.current.unwrap_or([pos[0], pos[1] - 1, pos[2]]);
            format!(
                "Scanning at {}, {}, {} (range: {} of {}).",
                at[0],
                at[1],
                at[2],
                scan.range.max(0),
                max_range
            )
        }
        PumpState::Full => "Internal tank is full.".to_string(),
        PumpState::Done => "Done.".to_string(),
    };
    let tank_line = format!(
        "{} {} / {} mB",
        tank.fluid.as_deref().unwrap_or("empty"),
        tank.amount,
        TANK_CAPACITY
    );
    (state_line, tank_line)
}

/// `/rangedpumps info`'s one-line summary of a pump.
fn info_line(nbt: &Compound, pos: Pos) -> String {
    let scan = read_scan(nbt);
    let max_range = configured_range(nbt);
    let tank = read_tank(nbt);
    let label = match PumpState::of(&scan, max_range, &tank) {
        PumpState::Working => "working",
        PumpState::Full => "full",
        PumpState::Done => "done",
    };
    let at = scan.current.unwrap_or([pos[0], pos[1] - 1, pos[2]]);
    format!(
        "Ranged Pump: {label}, range {} of {}, scan {}, {}, {}, {} {} / {} mB",
        scan.range.max(0),
        max_range,
        at[0],
        at[1],
        at[2],
        tank.fluid.as_deref().unwrap_or("empty"),
        tank.amount,
        TANK_CAPACITY
    )
}

/// Upstream `drainFluidAt(false)` + `tryReplaceLiquidWithBlock`: air first, then the configured
/// block (stone by default), so nearby fluid updates see the removal.
fn replace_source(host: &mut dyn ServerHost, level: LevelRef, pos: Pos) {
    let flags = BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS;
    if let Some(air) = host.block_state_id("minecraft:air") {
        let _ = level.set_block(host, pos, air, flags);
    }
    if let Some(stone) = host.block_state_id(REPLACE_WITH) {
        let _ = level.set_block(host, pos, stone, flags);
    }
}

/// Upstream `fillNeighbors`: an equal share of the tank into every adjacent fluid handler that
/// has room for it, one whole move per neighbour. Returns how much left the pump.
fn push_output(host: &mut dyn ServerHost, level: LevelRef, pos: Pos, tank: &Tank) -> i32 {
    let Some(fluid) = tank.fluid.as_deref() else {
        return 0;
    };
    if tank.amount <= 0 {
        return 0;
    }
    let Some(fluid_key) = host.fluid_key(fluid) else {
        return 0;
    };
    let resource = FluidResource::of(&FluidStack::new(fluid_key, 1));
    let mut targets: Vec<Capability<dyn ResourceHandler<FluidResource>>> = Vec::new();
    for face in Direction::ALL {
        let [dx, dy, dz] = face.normal();
        let neighbour = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
        if let Some(handler) = level.capability(host, &FLUIDS, neighbour, Some(face.opposite()))
            && (0..handler.size(host))
                .any(|index| handler.capacity(host, index, &resource) > handler.amount(host, index))
        {
            targets.push(handler);
        }
    }
    if targets.is_empty() {
        return 0;
    }
    let share = tank.amount / targets.len() as i32;
    if share <= 0 {
        return 0;
    }
    let mut moved = 0;
    for handler in targets {
        let went = host
            .transaction(|host, tx| {
                Ok::<u32, ()>(handler.insert_any(host, tx, &resource, share as u32))
            })
            .unwrap_or(0);
        moved += went as i32;
    }
    moved
}

/// One server tick of a pump: every `SPEED_TICKS`, advance the scan and drain a still source
/// block into the tank; then push the tank into the neighbours.
fn pump_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let Some(level) = in_place_level(host) else {
        return;
    };
    let pos = event.pos;
    let max_range = configured_range(&event.entity.nbt);
    let ticks = int(&event.entity.nbt, "ticks").unwrap_or(0).max(0) as u32;
    if ticks.is_multiple_of(SPEED_TICKS)
        && let Some((min_y, _height)) = level.height(host)
    {
        let mut scan = read_scan(&event.entity.nbt);
        // The probe walks a *copy*: a source the tank cannot take must leave the scan where it
        // was, so the next look retries that block. Advancing first (and saving the advance)
        // skipped every source behind a full tank for good.
        let mut next = scan;
        match next.advance(pos, min_y, max_range) {
            Some(target) => {
                let block = level.block(host, target).map(|block| block.id(host));
                let level_property = level.block_property(host, target, "level");
                match source_fluid(block.as_deref().unwrap_or(""), level_property.as_deref()) {
                    None => scan = next,
                    Some(fluid) => {
                        let mut tank = read_tank(&event.entity.nbt);
                        // `insert_bucket` is false when the tank is full or holds another
                        // fluid; the source stays in the world and the scan waits.
                        if tank.insert_bucket(fluid) {
                            write_tank(&mut event.entity.nbt, &tank);
                            replace_source(host, level, target);
                            scan = next;
                        }
                    }
                }
            }
            // The scan is finished: keep that, or it would start over.
            None => scan = next,
        }
        write_scan(&mut event.entity.nbt, &scan);
    }
    let tank = read_tank(&event.entity.nbt);
    let moved = push_output(host, level, pos, &tank);
    if moved > 0
        && let Some(fluid) = tank.fluid.as_deref()
    {
        write_tank(
            &mut event.entity.nbt,
            &Tank::new(fluid, tank.amount - moved),
        );
    }
    put(
        &mut event.entity.nbt,
        "ticks",
        Tag::Int((ticks as i32).wrapping_add(1)),
    );
}

/// The pump: its block entity (ticking, tank) and its block (the readout).
struct Pump;

#[mod_api::horizon_part]
impl BlockPart for Pump {
    /// A right click reports the state and the tank, as upstream's `useWithoutItem` does.
    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        let pos = event.click.pos;
        let level = in_place_level(host)?;
        let entity = pump_at(host, level, pos)?;
        let data = entity.data(host)?;
        let (state_line, tank_line) = readout(&data, pos);
        let player = host.player(event.click.player)?;
        let _ = player.tell(host, Text::literal(state_line));
        let _ = player.tell(host, Text::literal(tank_line));
        Some(InteractionResult::Success)
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for Pump {
    /// The pump ticks on the server (`EntityBlock.getTicker`).
    fn block_entity_ticks(&self, _host: &dyn mod_api::Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        pump_tick(host, event);
    }
}

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the pump's block entity, its tanks and its commands' arguments.
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        let block = r.block_key(PUMP);
        let kind = r.block_entity_type("pump").blocks(&[PUMP]).build();
        let _ = r.block_entity_part(&kind, Pump);
        // One 32-bucket tank, every face reaches it, and only output is allowed (upstream's
        // tank refuses external insertion and lets a neighbour drain it).
        let all = vec![0usize];
        let mut tanks = ModFluidTanks::default();
        tanks.tanks = vec![TANK_CAPACITY];
        tanks.faces = [
            all.clone(),
            all.clone(),
            all.clone(),
            all.clone(),
            all.clone(),
            all,
        ];
        tanks.inputs = Vec::new();
        tanks.outputs = vec![0];
        r.block_fluids(&block, tanks);
    }

    /// `/rangedpumps info|range`.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        event
            .commands
            .push(Command::new("rangedpumps info <pos:block_pos>"));
        event.commands.push(Command::op(
            "rangedpumps range <pos:block_pos> <range:integer>",
            2,
        ));
    }

    /// Run the `/rangedpumps` commands a player types.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        let invocation = event.command;
        let literals: Vec<&str> = invocation.literals.iter().map(String::as_str).collect();
        if !matches!(literals.as_slice(), ["rangedpumps", "info" | "range"]) {
            return Flow::Continue;
        }
        let Some(CommandArgumentValue::BlockPos(pos)) = invocation.argument("pos") else {
            feedback(
                host,
                "Usage: /rangedpumps info <x> <y> <z> or /rangedpumps range <x> <y> <z> <range>",
            );
            return Flow::Cancel;
        };
        let pos = *pos;
        let level = host
            .levels()
            .list_levels()
            .into_iter()
            .find(|level| level.dimension() == invocation.dimension);
        let Some(level) = level else {
            feedback(host, "No Ranged Pump there.");
            return Flow::Cancel;
        };
        match literals.as_slice() {
            ["rangedpumps", "info"] => {
                let Some(entity) = pump_at(host, level, pos) else {
                    feedback(host, "No Ranged Pump there.");
                    return Flow::Cancel;
                };
                let Some(data) = entity.data(host) else {
                    feedback(host, "No Ranged Pump there.");
                    return Flow::Cancel;
                };
                feedback(host, &info_line(&data, pos));
            }
            ["rangedpumps", "range"] => {
                let Some(CommandArgumentValue::Integer(range)) = invocation.argument("range")
                else {
                    feedback(host, "The range must be a whole number.");
                    return Flow::Cancel;
                };
                let Some(entity) = pump_at(host, level, pos) else {
                    feedback(host, "No Ranged Pump there.");
                    return Flow::Cancel;
                };
                let clamped = (*range).clamp(0, MAX_RANGE);
                let Some(mut data) = entity.data(host) else {
                    feedback(host, "No Ranged Pump there.");
                    return Flow::Cancel;
                };
                // Re-scan from the block below with the new maximum.
                put(&mut data, "configuredRange", Tag::Int(clamped));
                write_scan(&mut data, &Scan::new());
                put(&mut data, "ticks", Tag::Int(0));
                if entity.replace_data(host, data).is_err() {
                    feedback(host, "No Ranged Pump there.");
                    return Flow::Cancel;
                }
                feedback(host, &format!("Ranged Pump range set to {clamped}."));
            }
            _ => return Flow::Continue,
        }
        Flow::Cancel
    }
}

/// The command's success feedback, routed to its originating source (the swapped-in player, the
/// console, a command block) as the command service does.
fn feedback(host: &mut dyn ServerHost, line: &str) {
    let _ = host.commands().feedback(Text::literal(line));
}
