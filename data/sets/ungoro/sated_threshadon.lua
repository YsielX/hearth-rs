return {
    api_version = 1, id = "UNG_010", name = "Sated Threshadon",
    text = "<b>Taunt</b>\n<b>Deathrattle:</b> Summon three 1/1 Murlocs.",
    set = "UNGORO", type = "minion", rarity = "common", cost = 7,
    attack = 5, health = 8, tags = { "beast" }, keywords = { "taunt", "deathrattle" },
    on_deathrattle = function(ctx, self, position)
        for _ = 1, 3 do ctx:summon_at(ctx:controller(self), "UNG_201t", position) end
    end,
}
