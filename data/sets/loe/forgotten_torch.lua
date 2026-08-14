local card = {
    api_version = 1, id = "LOE_002", name = "Forgotten Torch",
    text = "Deal $3 damage. Shuffle a 'Roaring Torch' into your deck that deals 6 damage.",
    set = "LOE", type = "spell", class = "mage", rarity = "common",
    spell_school = "fire", cost = 3, target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
}

function card.on_play(ctx, self, target)
    ctx:damage(target, 3)
    ctx:shuffle_card_into_deck(ctx:controller(self), "LOE_002t")
end

card.tokens = {{
    id = "LOE_002t", name = "Roaring Torch", text = "Deal $6 damage.",
    set = "LOE", type = "spell", class = "mage", spell_school = "fire", cost = 3,
    target_mode = "required", targets = function(ctx) return ctx:characters() end,
    on_play = function(ctx, self, target) ctx:damage(target, 6) end,
}}

return card
