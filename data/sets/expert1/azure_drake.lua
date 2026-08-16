return {
    api_version = 1,
    id = "EX1_284",
    name = "Azure Drake",
    text = "<b>Spell Damage +1</b>\n<b>Battlecry:</b> Draw a card.",
    set = "EXPERT1",
    type = "minion",
    rarity = "rare",
    cost = 5,
    attack = 4,
    health = 5,
    tags = { "dragon" },
    keywords = { "spell_damage", "battlecry" },
    keyword_params = { spell_damage = 1 },
    on_battlecry = function(ctx, self)
        ctx:draw(ctx:controller(self), 1)
    end,
}
