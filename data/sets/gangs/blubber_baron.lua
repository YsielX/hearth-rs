local function has_battlecry(definition)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == "battlecry" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "CFM_064",
    name = "Blubber Baron",
    text = "Whenever you summon a <b>Battlecry</b> minion while this is in your hand, gain +1/+1.",
    set = "GANGS",
    type = "minion",
    rarity = "epic",
    cost = 3,
    attack = 1,
    health = 1,
    triggers = {{
        event = "minion_summoned", timing = "after", active_zones = { "hand" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
                and has_battlecry(ctx:card_definition(ctx:entity(event.entity).card_id))
        end,
        effect = function(ctx, self) ctx:buff(self, 1, 1) end,
    }},
}
