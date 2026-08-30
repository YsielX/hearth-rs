local function has_battlecry(definition)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == "battlecry" then return true end
    end
    return false
end

return {
    api_version = 1, id = "AT_121", name = "Crowd Favorite",
    text = "Whenever you play a card with <b>Battlecry</b>, gain +1/+1.", set = "TGT",
    type = "minion", rarity = "epic", cost = 4, attack = 4, health = 4,
    triggers = {{
        event = "card_played", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.entity ~= self
                and has_battlecry(ctx:card_definition(ctx:entity(event.entity).card_id))
        end,
        effect = function(ctx, self) cardlib.effects.buff(ctx, self, 1, 1) end,
    }},
}
