local card = {
    api_version = 1,
    id = "RLK_741",
    name = "Soulstealer",
    text = "[x]<b>Battlecry:</b> Destroy all other\nminions. Gain 1 <b>Corpse</b> for\neach enemy destroyed.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "death_knight",
    rarity = "epic",
    cost = 8,
    attack = 5,
    health = 5,
    rune_cost = { blood = 2 },
    tags = { "undead" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local targets = {}
    for _, minion in ipairs(ctx:minions()) do
        if minion ~= self then targets[#targets + 1] = minion end
    end
    ctx:set_data(self, "soulstealer_armed", 1)
    if #targets > 0 then cardlib.effects.destroy_all(ctx, targets) end
    ctx:continue_with("disarm_soulstealer")
end

function card.disarm_soulstealer(ctx, self)
    ctx:set_data(self, "soulstealer_armed", 0)
end

card.triggers = {{
    event = "entity_died",
    timing = "after",
    active_zones = { "board", "graveyard" },
    condition = function(ctx, self, event)
        local dead = ctx:entity(event.entity)
        return ctx:get_data(self, "soulstealer_armed") == 1
            and event.source == self
            and dead.type == "minion"
            and dead.controller ~= ctx:controller(self)
    end,
    effect = function(ctx, self)
        ctx:gain_resource(ctx:controller(self), "corpses", 1)
    end,
}}

return card
