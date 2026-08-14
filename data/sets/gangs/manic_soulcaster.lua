return {
    api_version = 1,
    id = "CFM_660",
    name = "Manic Soulcaster",
    text = "<b>Battlecry:</b> Choose a friendly minion. Shuffle a copy into your deck.",
    set = "GANGS",
    type = "minion",
    class = "mage",
    rarity = "epic",
    cost = 3,
    attack = 3,
    health = 4,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self then result[#result + 1] = minion end
        end
        return result
    end,
    on_battlecry = function(ctx, self, target)
        if target then
            ctx:shuffle_card_into_deck(ctx:controller(self), ctx:entity(target).card_id)
        end
    end,
}
