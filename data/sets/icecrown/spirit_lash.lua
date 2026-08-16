local card = {
    api_version = 1, id = "ICC_802", name = "Spirit Lash",
    text = "<b>Lifesteal</b>\nDeal $1 damage to all minions.", set = "ICECROWN",
    type = "spell", class = "priest", rarity = "common", spell_school = "shadow",
    cost = 2, keywords = { "lifesteal" },
}

function card.on_play(ctx, self) cardlib.effects.damage_all(ctx, ctx:minions(), 1) end

return card
