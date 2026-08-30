local card = {
    api_version = 1, id = "OG_051", name = "Forbidden Ancient",
    text = "<b>Battlecry:</b> Spend all your Mana. Gain +1/+1 for each Mana spent.",
    set = "OG", type = "minion", class = "druid", rarity = "epic",
    cost = 1, attack = 1, health = 1, keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local amount = ctx:player(player).mana
    ctx:spend_mana(player, amount)
    if amount > 0 then cardlib.effects.buff(ctx, self, amount, amount) end
end
return card
