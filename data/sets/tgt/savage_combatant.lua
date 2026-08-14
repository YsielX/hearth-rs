return {
    api_version = 1, id = "AT_039", name = "Savage Combatant",
    text = "<b>Inspire:</b> Give your hero +2 Attack this turn.",
    set = "TGT", type = "minion", class = "druid", rarity = "rare",
    cost = 4, attack = 5, health = 4, tags = { "beast" }, keywords = { "inspire" },
    on_inspire = function(ctx, self)
        ctx:buff_until_end_of_turn(ctx:player(ctx:controller(self)).hero, 2, 0)
    end,
}
