local card = {
    api_version = 1, id = "OG_045", name = "Infest",
    text = "Give your minions \"<b>Deathrattle:</b> Add a random Beast to your hand.\"",
    set = "OG", type = "spell", class = "hunter", rarity = "rare",
    spell_school = "nature", cost = 3,
}
function card.on_play(ctx, self)
    for _, minion in ipairs(ctx:board(ctx:controller(self))) do
        local entity = ctx:entity(minion)
        local dormant = false
        for _, keyword in ipairs(entity.keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if entity.type == "minion" and not dormant then
            ctx:attach_hook(minion, "on_deathrattle", "OG_045")
            cardlib.effects.grant_keyword(ctx, minion, "deathrattle")
        end
    end
end
function card.on_deathrattle(ctx, self)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        for _, tag in ipairs(ctx:card_definition(card_id).tags) do
            if tag == "beast" or tag == "all" then pool[#pool + 1] = card_id break end
        end
    end
    if #pool > 0 then ctx:random_value(pool, "receive_random_beast") end
end
function card.receive_random_beast(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end
return card
