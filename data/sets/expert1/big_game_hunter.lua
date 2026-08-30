return {
    api_version = 1,
    id = "EX1_005", rarity = "epic",
    name = "Big Game Hunter",
    text = "[x]<b>Tradeable</b>\n<b>Battlecry:</b> Destroy a minion\nwith 7 or more Attack.",
    set = "EXPERT1",
    type = "minion",
    cost = 4,
    attack = 4,
    health = 2,
    keywords = { "tradeable", "battlecry" },
    target_mode = "required_if_available",

    targets = function(ctx, self)
        local result = {}
        for _, entity in ipairs(ctx:minions()) do
            if ctx:entity(entity).attack >= 7 then
                result[#result + 1] = entity
            end
        end
        return result
    end,

    on_battlecry = function(ctx, self, target)
        if target ~= nil then
            cardlib.effects.destroy(ctx, target)
        end
    end,
}
