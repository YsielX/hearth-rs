return {
    api_version = 1,
    id = "EX1_012",
    name = "Bloodmage Thalnos",
    text = "<b>Spell Damage +1</b>\n<b>Deathrattle:</b> Draw a card.",
    set = "EXPERT1",
    type = "minion",
    rarity = "legendary",
    class = "neutral",
    tags = { "undead" },
    cost = 2,
    attack = 1,
    health = 1,
    keywords = { "spell_damage", "deathrattle" },
    keyword_params = { spell_damage = 1 },
    on_deathrattle = function(ctx, self)
        ctx:draw(ctx:controller(self), 1)
    end,
}
