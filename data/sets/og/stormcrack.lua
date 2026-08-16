return {
    api_version = 1, id = "OG_206", name = "Stormcrack",
    text = "Deal $4 damage to a minion. <b>Overload:</b> (1)", set = "OG", type = "spell",
    class = "shaman", rarity = "common", cost = 2, spell_school = "nature",
    keywords = { "overload" }, keyword_params = { overload = 1 },
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target) cardlib.effects.damage(ctx, target, 4) end,
}
