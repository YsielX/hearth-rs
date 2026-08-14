local function has_keyword(definition, wanted)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == wanted then return true end
    end
    return false
end

return {
    api_version = 1, id = "ICC_086", name = "Glacial Mysteries",
    text = "Put one of each <b>Secret</b> from your deck into\nthe battlefield.",
    set = "ICECROWN", type = "spell", class = "mage", rarity = "epic",
    spell_school = "frost", cost = 8,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        local present, count = {}, #ctx:secrets(player)
        for _, secret in ipairs(ctx:secrets(player)) do present[ctx:entity(secret).card_id] = true end
        for _, entity in ipairs(ctx:deck(player)) do
            local definition = ctx:card_definition(ctx:entity(entity).card_id)
            if count < 5 and has_keyword(definition, "secret") and not present[definition.id] then
                present[definition.id] = true
                count = count + 1
                ctx:move(entity, "secret")
            end
        end
    end,
}
