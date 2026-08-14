local card = {
    api_version = 1, id = "ICC_823", name = "Simulacrum",
    text = "Copy the lowest Cost minion in your hand.",
    set = "ICECROWN", type = "spell", class = "mage", rarity = "epic",
    spell_school = "frost", cost = 3,
}

function card.on_play(ctx, self)
    local lowest, candidates = nil, {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        local snapshot = ctx:entity(entity)
        if snapshot.type == "minion" then
            if lowest == nil or snapshot.cost < lowest then
                lowest, candidates = snapshot.cost, { entity }
            elseif snapshot.cost == lowest then
                candidates[#candidates + 1] = entity
            end
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "simulacrum_chosen") end
end

function card.simulacrum_chosen(ctx, self, entity)
    ctx:give_copy(ctx:controller(self), entity)
end

return card
