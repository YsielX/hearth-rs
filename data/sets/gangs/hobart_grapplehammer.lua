local card = {
    api_version = 1, id = "CFM_643", name = "Hobart Grapplehammer",
    text = "[x]<b>Battlecry:</b> If you have a\nweapon equipped, give\nall minions in your hand\nand deck +2/+2.",
    set = "GANGS", type = "minion", class = "warrior", rarity = "legendary",
    cost = 4, attack = 2, health = 2, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    if not ctx:player(player).weapon then return end
    for _, zone in ipairs({ ctx:hand(player), ctx:deck(player) }) do
        for _, entity in ipairs(zone) do
            if ctx:entity(entity).type == "minion" then ctx:buff(entity, 2, 2) end
        end
    end
end
return card
