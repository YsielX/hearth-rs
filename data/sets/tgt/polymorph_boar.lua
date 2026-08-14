local card = {
    api_version = 1, id = "AT_005", name = "Polymorph: Boar",
    text = "Transform a minion into a 4/2 Boar with <b>Charge</b>.", set = "TGT",
    type = "spell", class = "mage", rarity = "rare", spell_school = "arcane",
    cost = 3, target_mode = "required", targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target) ctx:transform(target, "AT_005t") end,
}

card.tokens = {
    {
        id = "AT_005t", name = "Boar", text = "<b>Charge</b>", set = "TGT",
        type = "minion", class = "mage", cost = 3, attack = 4, health = 2,
        tags = { "beast" }, keywords = { "charge" },
    },
}

return card
