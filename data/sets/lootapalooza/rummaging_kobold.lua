local card = {
    api_version = 1,
    id = "LOOT_389",
    name = "Rummaging Kobold",
    text = "<b>Battlecry:</b> Return one of your destroyed weapons to your hand.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "epic",
    cost = 3,
    attack = 1,
    health = 3,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local weapons = ctx:weapons_destroyed(ctx:controller(self))
    if #weapons > 0 then ctx:random_value(weapons, "rummaging_weapon_chosen") end
end

function card.rummaging_weapon_chosen(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

return card
