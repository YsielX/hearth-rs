local card = {
    api_version = 1, id = "LOOT_333", name = "Level Up!",
    text = "Give your Silver Hand Recruits +2/+2 and <b>Taunt</b>.", set = "LOOTAPALOOZA",
    type = "spell", class = "paladin", rarity = "epic", cost = 5,
}
function card.on_play(ctx, self)
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if ctx:entity(minion).card_id == "CS2_101t" then
            ctx:buff(minion, 2, 2); ctx:grant_keyword(minion, "taunt")
        end
    end
end
return card
