return {
    api_version = 1, id = "ICC_067", name = "Vryghoul",
    text = "[x]<b>Deathrattle:</b> If it's your\nopponent's turn,\nsummon a 2/2 Ghoul.",
    set = "ICECROWN", type = "minion", rarity = "common",
    cost = 3, attack = 3, health = 1, tags = { "undead" }, keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local player = ctx:controller(self)
        if ctx:active_player() ~= player then ctx:summon(player, "ICC_900t") end
    end,
}
