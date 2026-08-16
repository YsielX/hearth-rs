return {
    api_version = 1,
    id = "LOE_017",
    name = "Keeper of Uldaman",
    text = "<b>Battlecry:</b> Set a minion's Attack and Health to 3.",
    set = "LOE",
    type = "minion",
    class = "paladin",
    rarity = "common",
    cost = 3,
    attack = 3,
    health = 4,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx) return ctx:minions() end,
    on_battlecry = function(ctx, self, target)
        if target == nil then return end
        cardlib.effects.modify(ctx, target, { stat = "attack", operation = "set", value = 3 })
        ctx:set_health(target, 3)
    end,
}
