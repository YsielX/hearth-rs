return {
    api_version = 1,
    id = "ICC_901",
    name = "Drakkari Enchanter",
    text = "Your end of turn effects trigger twice.",
    set = "ICECROWN",
    type = "minion",
    rarity = "epic",
    cost = 3,
    attack = 1,
    health = 5,
    auras = {
        {
            targets = function(ctx, self)
                return { ctx:player(ctx:controller(self)).hero }
            end,
            keywords = { "end_of_turn_repeater" },
        },
    },
}
