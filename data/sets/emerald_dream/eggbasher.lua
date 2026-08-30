return {
    api_version = 1,
    id = "EDR_468", rarity = "common",
    name = "Eggbasher",
    text = "<b>Battlecry:</b> Deal 1 damage to a minion and give it\n+4 Attack.",
    set = "EMERALD_DREAM",
    type = "minion",
    class = "warrior",
    cost = 4,
    attack = 3,
    health = 5,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self) return ctx:minions() end,
    on_battlecry = function(ctx, self, target)
        if target == nil then return end
        cardlib.effects.damage(ctx, target, 1)
        cardlib.effects.buff(ctx, target, 4, 0)
    end,
}
