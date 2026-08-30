local card = {
    api_version = 1,
    id = "RLK_730",
    name = "Blood Boil",
    text = "<b>Lifesteal</b>\nInfect all enemy minions. At the end of your turns, they take 2 damage.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "epic",
    spell_school = "shadow",
    cost = 5,
    rune_cost = { blood = 2 },
    keywords = { "lifesteal" },
}

function card.on_play(ctx, self)
    for _, minion in ipairs(ctx:enemy_minions(self)) do
        -- A zero-stat enchantment is a silenciable, source-specific infection marker.
        ctx:buff(minion, 0, 0)
    end
end

card.triggers = {{
    event = "turn_ended",
    timing = "after",
    active_zones = { "graveyard" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self)
    end,
    effect = function(ctx, self)
        local hits = {}
        for _, minion in ipairs(ctx:minions()) do
            if ctx:has_enchantment_from(minion, self) then
                hits[#hits + 1] = { minion, 2 }
            end
        end
        if #hits > 0 then cardlib.effects.damage_batch(ctx, hits) end
    end,
}}

return card
