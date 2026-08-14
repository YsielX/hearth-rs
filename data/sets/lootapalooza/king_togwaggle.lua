local ransom = {
    id = "LOOT_541t",
    name = "King's Ransom",
    text = "Swap decks with your opponent.",
    set = "LOOTAPALOOZA",
    type = "spell",
    collectible = false,
    cost = 5,
}

function ransom.on_play(ctx, self)
    local player = ctx:controller(self)
    ctx:exchange_zone_contents(player, ctx:opponent(player), "deck")
end

local card = {
        api_version = 1,
        id = "LOOT_541",
        name = "King Togwaggle",
        text = "[x]<b>Battlecry:</b> Swap decks\nwith your opponent.\nGive them a Ransom\nspell to swap back.",
        set = "LOOTAPALOOZA",
        type = "minion",
        rarity = "legendary",
        cost = 8,
        attack = 5,
        health = 5,
        keywords = { "battlecry" },
        tokens = { ransom },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local opponent = ctx:opponent(player)
    ctx:exchange_zone_contents(player, opponent, "deck")
    ctx:give_card(opponent, "LOOT_541t")
end

return card
