local card = {
    api_version = 1, id = "CFM_094", name = "Felfire Potion",
    text = "Deal $5 damage to all characters.", set = "GANGS", type = "spell",
    class = "warlock", spell_school = "fel", rarity = "rare", cost = 6,
}
function card.on_play(ctx, self) ctx:damage_all(ctx:characters(), 5) end
return card
