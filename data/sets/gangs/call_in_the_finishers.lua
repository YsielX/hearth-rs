local card = {
    api_version = 1, id = "CFM_310", name = "Call in the Finishers",
    text = "Summon four\n1/1 Murlocs.", set = "GANGS", type = "spell",
    class = "shaman", rarity = "common", cost = 3,
    rules = { can_play = function(ctx, self) return #ctx:board(ctx:controller(self)) < 7 end },
}
function card.on_play(ctx, self)
    for _ = 1, 4 do ctx:summon(ctx:controller(self), "CFM_310t") end
end
card.tokens = {{
    id = "CFM_310t", name = "Murloc Razorgill", text = "", set = "GANGS",
    type = "minion", class = "shaman", cost = 1, attack = 1, health = 1, tags = { "murloc" },
}}
return card
