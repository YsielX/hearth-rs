local card = { api_version = 1, id = "UNG_027", name = "Pyros",
    text = "<b>Deathrattle:</b> Return this to your hand as a 6/6 that costs (4).", set = "UNGORO",
    type = "minion", class = "mage", rarity = "legendary", cost = 2, attack = 2, health = 2,
    tags = { "elemental", "beast" }, keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self) cardlib.effects.give_card(ctx, ctx:controller(self), "UNG_027t2") end }
card.tokens = {
    { id = "UNG_027t2", name = "Pyros", text = "<b>Deathrattle:</b> Return this to your hand as a 10/10 that costs (8).",
      set = "UNGORO", type = "minion", class = "mage", rarity = "legendary", collectible = false,
      cost = 4, attack = 6, health = 6, tags = { "elemental", "beast" }, keywords = { "deathrattle" },
      on_deathrattle = function(ctx, self) cardlib.effects.give_card(ctx, ctx:controller(self), "UNG_027t4") end },
    { id = "UNG_027t4", name = "Pyros", text = "", set = "UNGORO", type = "minion", class = "mage",
      rarity = "legendary", collectible = false, cost = 8, attack = 10, health = 10, tags = { "elemental", "beast" } },
}
return card
