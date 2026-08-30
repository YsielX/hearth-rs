local card = {
    api_version = 1, id = "UNG_029", name = "Shadow Visions",
    text = "<b>Discover</b> a copy of a spell in your deck.",
    set = "UNGORO", type = "spell", class = "priest", rarity = "epic", spell_school = "shadow", cost = 2,
}
function card.on_play(ctx, self)
    local spells = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        if ctx:entity(entity).type == "spell" then spells[#spells + 1] = entity end
    end
    if #spells > 0 then
        ctx:discover_entities(ctx:controller(self), "Choose a spell to copy", spells, 3, "copy_shadow_vision")
    end
end
function card.copy_shadow_vision(ctx, self, entity)
    cardlib.effects.give_card(ctx, ctx:controller(self), ctx:entity(entity).card_id)
end
return card
