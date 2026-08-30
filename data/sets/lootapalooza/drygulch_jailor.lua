local card = {
    api_version = 1, id = "LOOT_363", name = "Drygulch Jailor",
    text = "<b>Deathrattle:</b> Add 3 Silver Hand Recruits to your hand.",
    set = "LOOTAPALOOZA", type = "minion", class = "paladin", rarity = "common",
    cost = 2, attack = 1, health = 1, keywords = { "deathrattle" },
}
function card.on_deathrattle(ctx, self)
    local player = ctx:controller(self)
    cardlib.effects.give_card(ctx, player, "CS2_101t"); cardlib.effects.give_card(ctx, player, "CS2_101t"); cardlib.effects.give_card(ctx, player, "CS2_101t")
end
return card
