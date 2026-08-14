return {
    api_version = 1,
    id = "FP1_021",
    name = "Death's Bite",
    text = "<b>Deathrattle:</b> Deal 1 damage to all minions.",
    set = "NAXX",
    type = "weapon",
    class = "warrior",
    rarity = "common",
    cost = 4,
    attack = 4,
    health = 2,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        ctx:damage_all(ctx:minions(), 1)
    end,
}
