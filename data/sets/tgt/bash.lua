return {
    api_version = 1, id = "AT_064", name = "Bash", text = "Deal $3 damage.\nGain 3 Armor.",
    set = "TGT", type = "spell", class = "warrior", rarity = "common", cost = 2,
    target_mode = "required", targets = function(ctx) return ctx:characters() end,
    on_play = function(ctx, self, target)
        cardlib.effects.damage(ctx, target, 3)
        ctx:gain_armor(ctx:controller(self), 3)
    end,
}
