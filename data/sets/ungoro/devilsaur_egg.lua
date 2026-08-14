return {
    api_version = 1, id = "UNG_083", name = "Devilsaur Egg",
    text = "<b>Deathrattle:</b> Summon a 5/5 Devilsaur.", set = "UNGORO",
    type = "minion", rarity = "rare", cost = 3, attack = 0, health = 3,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self, position) ctx:summon_at(ctx:controller(self), "UNG_083t1", position) end,
    tokens = {{ id = "UNG_083t1", name = "Devilsaur", text = "", set = "UNGORO",
        type = "minion", cost = 5, attack = 5, health = 5, tags = { "beast" } }},
}
