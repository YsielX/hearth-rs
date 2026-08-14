local KEY = "next_secret_cost_one_this_turn"

return {
    api_version = 1,
    id = "CFM_066",
    name = "Kabal Lackey",
    text = "[x]<b>Battlecry:</b> The next <b>Secret</b>\nyou play this turn costs (1).",
    set = "GANGS",
    type = "minion",
    class = "mage",
    rarity = "common",
    cost = 1,
    attack = 2,
    health = 1,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        ctx:set_player_data(player, KEY, 1)
        ctx:grant_player_keyword(player, KEY)
    end,
}
