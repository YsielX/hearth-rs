local function has(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do if keyword == wanted then return true end end
    return false
end
local card = {
    api_version = 1, id = "UNG_900", name = "Spiritsinger Umbra",
    text = "After you summon a minion, trigger its <b>Deathrattle</b> effect.",
    set = "UNGORO", type = "minion", rarity = "legendary", cost = 5, attack = 3, health = 4,
    triggers = {{
        event = "minion_summoned", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
                and event.entity ~= self
                and ctx:entity(event.entity).zone == "board"
                and has(ctx, event.entity, "deathrattle")
        end,
        effect = function(ctx, self, event)
            local repeats = has(ctx, event.entity, "deathrattle_repeater") and 2 or 1
            for _ = 1, repeats do ctx:trigger_hook(event.entity, "on_deathrattle", ctx:board_position(event.entity)) end
        end,
    }},
}
return card
