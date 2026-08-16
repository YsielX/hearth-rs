return {
    api_version = 1,
    id = "CFM_335",
    name = "Dispatch Kodo",
    text = "<b>Battlecry:</b> Deal damage equal to this minion's Attack.",
    set = "GANGS",
    type = "minion",
    class = "hunter",
    rarity = "rare",
    cost = 4,
    attack = 2,
    health = 4,
    tags = { "beast" },
    keywords = { "battlecry" },
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
    on_battlecry = function(ctx, self, target)
        cardlib.effects.damage(ctx, target, ctx:entity(self).attack)
    end,
}
