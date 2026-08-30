local card = {
    api_version = 1, id = "CFM_810", name = "Leatherclad Hogleader",
    text = "<b>Battlecry:</b> If your opponent has 6 or more cards in hand, gain <b>Charge</b>.",
    set = "GANGS", type = "minion", rarity = "epic", cost = 6, attack = 6,
    health = 6, tags = { "quilboar" }, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    if #ctx:hand(ctx:opponent(ctx:controller(self))) >= 6 then cardlib.effects.grant_keyword(ctx, self, "charge") end
end
return card
