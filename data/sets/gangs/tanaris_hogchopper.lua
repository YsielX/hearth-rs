local card = {
    api_version = 1, id = "CFM_809", name = "Tanaris Hogchopper",
    text = "[x]<b>Battlecry:</b> If your opponent's\nhand is empty, gain <b>Charge</b>.",
    set = "GANGS", type = "minion", rarity = "common", cost = 4, attack = 4,
    health = 4, tags = { "quilboar" }, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    if #ctx:hand(ctx:opponent(ctx:controller(self))) == 0 then cardlib.effects.grant_keyword(ctx, self, "charge") end
end
return card
