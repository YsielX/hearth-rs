local card = {
    api_version = 1, id = "OG_109", name = "Darkshire Librarian",
    text = "<b>Battlecry:</b>\nDiscard a random card. <b>Deathrattle:</b>\nDraw a card.", set = "OG",
    type = "minion", class = "warlock", rarity = "rare", cost = 2, attack = 3, health = 2,
    keywords = { "battlecry", "deathrattle" },
}
function card.on_battlecry(ctx, self)
    local hand = ctx:hand(ctx:controller(self))
    if #hand > 0 then ctx:random_entity(hand, "discard_random_card") end
end
function card.discard_random_card(ctx, self, target)
    ctx:discard(ctx:controller(self), target)
end
function card.on_deathrattle(ctx, self) ctx:draw(ctx:controller(self), 1) end
return card
