local card = {
    api_version = 1,
    id = "NX2_035",
    name = "Rimescale Siren",
    text = "[x]<b>Battlecry:</b> If you've cast three\nspells while holding this,\n<b>Freeze</b> 3 random enemy\nminions.@ <i>({0} left!)</i>@ <i>(Ready!)</i>",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "death_knight",
    rarity = "common",
    cost = 3,
    attack = 2,
    health = 3,
    rune_cost = { frost = 1 },
    tags = { "undead", "naga" },
    keywords = { "battlecry" },
}

card.triggers = {{
    event = "spell_cast",
    timing = "after",
    active_zones = { "hand" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and event.player_cast
    end,
    effect = function(ctx, self)
        ctx:set_data(self, "rimescale_spells", ctx:get_data(self, "rimescale_spells") + 1)
    end,
}}

function card.on_battlecry(ctx, self)
    if ctx:get_data(self, "rimescale_spells") < 3 then return end
    ctx:set_data(self, "rimescale_left", math.min(3, #ctx:enemy_minions(self)))
    ctx:continue_with("freeze_next_rimescale_target")
end

function card.freeze_next_rimescale_target(ctx, self)
    if ctx:get_data(self, "rimescale_left") <= 0 then return end
    local candidates = {}
    for _, minion in ipairs(ctx:enemy_minions(self)) do
        if ctx:get_data(self, "rimescale_frozen_" .. minion) == 0 then
            candidates[#candidates + 1] = minion
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "freeze_rimescale_target") end
end

function card.freeze_rimescale_target(ctx, self, target)
    ctx:set_data(self, "rimescale_frozen_" .. target, 1)
    ctx:set_data(self, "rimescale_left", ctx:get_data(self, "rimescale_left") - 1)
    ctx:freeze(target)
    ctx:continue_with("freeze_next_rimescale_target")
end

return card
