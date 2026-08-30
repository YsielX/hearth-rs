return {
    api_version = 1, id = "UNG_076", name = "Eggnapper",
    text = "<b>Deathrattle:</b> Summon two 1/1 Raptors.", set = "UNGORO",
    type = "minion", rarity = "common", cost = 3, attack = 3, health = 1,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self, position)
        cardlib.effects.summon_at(ctx, ctx:controller(self), "UNG_076t1", position)
        cardlib.effects.summon_at(ctx, ctx:controller(self), "UNG_076t1", position)
    end,
    tokens = {{ id = "UNG_076t1", name = "Raptor", text = "", set = "UNGORO",
        type = "minion", cost = 1, attack = 1, health = 1, tags = { "beast" } }},
}
