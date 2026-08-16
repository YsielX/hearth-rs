local card = {
    api_version = 1, id = "AT_051", name = "Elemental Destruction",
    text = "Deal $4-$5 damage to all minions. <b>Overload:</b> (2)", set = "TGT", type = "spell",
    class = "shaman", rarity = "epic", cost = 3, spell_school = "nature",
    keywords = { "overload" }, keyword_params = { overload = 2 },
}

function card.on_play(ctx, self)
    ctx:random_value({ 4, 5 }, "deal_elemental_damage")
end

function card.deal_elemental_damage(ctx, self, amount)
    cardlib.effects.damage_all(ctx, ctx:minions(), amount)
end

return card
