local card = {
    api_version = 1, id = "ICC_085", name = "Ultimate Infestation",
    text = "[x]Deal $5 damage. Draw\n5 cards. Gain 5 Armor.\nSummon a 5/5 Ghoul.",
    set = "ICECROWN", type = "spell", class = "druid", rarity = "epic", cost = 10,
    target_mode = "required", targets = function(ctx) return ctx:characters() end,
}

function card.on_play(ctx, self, target)
    local player = ctx:controller(self)
    ctx:damage(target, 5)
    ctx:draw(player, 5)
    ctx:gain_armor(player, 5)
    ctx:summon(player, "ICC_085t")
end

card.tokens = {{
    id = "ICC_085t", name = "Ghoul Infestor", text = "", set = "ICECROWN",
    type = "minion", class = "druid", collectible = false,
    cost = 5, attack = 5, health = 5, tags = { "undead" },
}}

return card
