local card = {
    api_version = 1, id = "ICC_025", name = "Rattling Rascal",
    text = "[x]<b>Battlecry:</b> Summon a\n5/5 Skeleton.\n<b>Deathrattle:</b> Summon one\nfor your opponent.",
    set = "ICECROWN", type = "minion", rarity = "epic",
    cost = 4, attack = 2, health = 2, tags = { "undead" }, keywords = { "battlecry", "deathrattle" },
}

function card.on_battlecry(ctx, self) ctx:summon(ctx:controller(self), "ICC_025t") end
function card.on_deathrattle(ctx, self) ctx:summon(ctx:opponent(ctx:controller(self)), "ICC_025t") end

card.tokens = {{
    id = "ICC_025t", name = "Skeletal Enforcer", text = "", set = "ICECROWN",
    type = "minion", collectible = false, cost = 5, attack = 5, health = 5, tags = { "undead" },
}}

return card
