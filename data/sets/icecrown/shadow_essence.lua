local card = {
    api_version = 1, id = "ICC_235", name = "Shadow Essence",
    text = "Summon a 5/5 copy of a random minion in your deck.", set = "ICECROWN",
    type = "spell", class = "priest", rarity = "rare", spell_school = "shadow", cost = 7,
}

function card.on_play(ctx, self)
    local pool = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" then pool[#pool + 1] = entity end
    end
    if #pool > 0 then ctx:random_entity(pool, "summon_shadow_essence") end
end

function card.summon_shadow_essence(ctx, self, entity)
    cardlib.effects.summon_copy_with_stats(ctx, ctx:controller(self), entity, 5, 5)
end

return card
