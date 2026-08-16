local card = {
    api_version = 1,
    id = "GVG_038",
    name = "Crackle",
    text = "Deal $3-$6 damage. <b>Overload:</b> (1)",
    set = "GVG",
    type = "spell",
    class = "shaman",
    rarity = "common",
    spell_school = "nature",
    cost = 2,
    keywords = { "overload" },
    keyword_params = { overload = 1 },
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
}

function card.on_play(ctx, self, target)
    ctx:set_data(self, "target", target)
    ctx:random_value({ 3, 4, 5, 6 }, "deal_crackle_damage")
end

function card.deal_crackle_damage(ctx, self, amount)
    cardlib.effects.damage(ctx, ctx:get_data(self, "target"), amount)
end

return card
