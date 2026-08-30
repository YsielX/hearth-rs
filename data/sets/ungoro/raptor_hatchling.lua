return { api_version = 1, id = "UNG_914", name = "Raptor Hatchling",
    text = "<b>Deathrattle:</b> Shuffle a 4/5 Raptor into your deck.", set = "UNGORO",
    type = "minion", class = "hunter", rarity = "rare", cost = 1, attack = 2, health = 1,
    tags = { "beast" }, keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self) cardlib.effects.shuffle_card_into_deck(ctx, ctx:controller(self), "UNG_914t1") end,
    tokens = {{ id = "UNG_914t1", name = "Raptor Patriarch", text = "", set = "UNGORO",
        type = "minion", class = "hunter", cost = 1, attack = 4, health = 5, tags = { "beast" } }} }
