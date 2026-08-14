return { api_version = 1, id = "UNG_018", name = "Flame Geyser",
    text = "Deal $2 damage.\nAdd a 1/2 Elemental to your hand.", set = "UNGORO",
    type = "spell", class = "mage", rarity = "common", spell_school = "fire", cost = 1,
    target_mode = "required", targets = function(ctx) return ctx:characters() end,
    on_play = function(ctx, self, target)
        ctx:damage(target, 2)
        ctx:give_card(ctx:controller(self), "UNG_809t1")
    end }
