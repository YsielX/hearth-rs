return {
    api_version = 1, id = "AT_062", name = "Ball of Spiders",
    text = "Summon three 1/1 Webspinners\nwith \"<b>Deathrattle:</b> Get\na random Beast.\"",
    set = "TGT", type = "spell", class = "hunter", rarity = "rare", cost = 3,
    rules = {
        can_play = function(ctx, self, current)
            return current and #ctx:board(ctx:controller(self)) < 7
        end,
    },
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        for _ = 1, 3 do ctx:summon(player, "FP1_011") end
    end,
}
