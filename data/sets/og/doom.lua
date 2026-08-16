local card = {
    api_version = 1, id = "OG_239", name = "DOOM!",
    text = "Destroy all minions. Draw a card for each.", set = "OG", type = "spell",
    class = "warlock", rarity = "epic", cost = 10, spell_school = "shadow",
}
function card.on_play(ctx, self)
    local minions = {}
    for _, minion in ipairs(ctx:minions()) do
        local dormant = false
        for _, keyword in ipairs(ctx:entity(minion).keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if not dormant then minions[#minions + 1] = minion end
    end
    ctx:set_data(self, "doom_draws", #minions)
    cardlib.effects.destroy_all(ctx, minions)
    ctx:continue_with("draw_doom_cards")
end
function card.draw_doom_cards(ctx, self)
    ctx:draw(ctx:controller(self), ctx:get_data(self, "doom_draws") or 0)
end
return card
