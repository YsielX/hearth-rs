return {
    api_version = 1,
    id = "FP1_024",
    name = "Unstable Ghoul",
    text = "<b>Taunt</b>. <b>Deathrattle:</b> Deal 1 damage to all minions.",
    set = "NAXX",
    type = "minion",
    rarity = "common",
    cost = 2,
    attack = 1,
    health = 3,
    tags = { "undead" },
    keywords = { "taunt", "deathrattle" },
    on_deathrattle = function(ctx, self)
        cardlib.effects.damage_all(ctx, ctx:minions(), 1)
    end,
}
