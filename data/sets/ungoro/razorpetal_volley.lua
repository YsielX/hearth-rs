return {
    api_version = 1, id = "UNG_057", name = "Razorpetal Volley",
    text = "Add two Razorpetals to your hand that deal 2 damage.",
    set = "UNGORO", type = "spell", class = "rogue", rarity = "common", spell_school = "nature", cost = 2,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        cardlib.effects.give_card(ctx, player, "UNG_057t1")
        cardlib.effects.give_card(ctx, player, "UNG_057t1")
    end,
    tokens = {{
        id = "UNG_057t1", name = "Razorpetal", text = "Deal $2 damage.",
        set = "UNGORO", type = "spell", class = "rogue", spell_school = "nature", cost = 1,
        target_mode = "required", targets = function(ctx, self) return ctx:characters() end,
        on_play = function(ctx, self, target) cardlib.effects.damage(ctx, target, 2) end,
    }},
}
