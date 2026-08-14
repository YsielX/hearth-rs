return {
    api_version = 1, id = "AT_006", name = "Dalaran Aspirant",
    text = "<b>Spell Damage +1</b>\n<b>Inspire:</b> Gain <b>Spell Damage +1</b>.",
    set = "TGT", type = "minion", class = "mage", rarity = "common",
    cost = 4, attack = 3, health = 5,
    keywords = { "spell_damage", "inspire" }, keyword_params = { spell_damage = 1 },
    on_inspire = function(ctx, self)
        ctx:modify(self, { stat = "spell_damage", operation = "add", value = 1 })
    end,
}
