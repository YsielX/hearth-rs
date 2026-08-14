local card = {
    api_version = 1, id = "UNG_032", name = "Crystalline Oracle",
    text = "[x]<b>Deathrattle:</b> Copy a card\nfrom your opponent's deck\n and add it to your hand.",
    set = "UNGORO", type = "minion", class = "priest", rarity = "rare",
    cost = 1, attack = 1, health = 2, tags = { "elemental" }, keywords = { "deathrattle" },
}
function card.on_deathrattle(ctx, self)
    local deck = ctx:deck(ctx:opponent(ctx:controller(self)))
    if #deck > 0 then ctx:random_entity(deck, "copy_oracle_card") end
end
function card.copy_oracle_card(ctx, self, entity)
    ctx:give_card(ctx:controller(self), ctx:entity(entity).card_id)
end
return card
