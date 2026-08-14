local card = {
    api_version = 1, id = "UNG_840", name = "Hemet, Jungle Hunter",
    text = "<b>Battlecry:</b> Destroy all cards in your deck that cost (3) or less.",
    set = "UNGORO", type = "minion", rarity = "legendary", cost = 6, attack = 6, health = 6,
    keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        if ctx:entity(entity).cost <= 3 then ctx:move(entity, "graveyard") end
    end
end
return card
