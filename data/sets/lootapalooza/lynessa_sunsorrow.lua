local card = {
    api_version = 1, id = "LOOT_216", name = "Lynessa Sunsorrow",
    text = "<b>Battlecry:</b> Cast each spell you cast on your minions this game on this one.",
    set = "LOOTAPALOOZA", type = "minion", class = "paladin", rarity = "legendary",
    cost = 7, attack = 1, health = 1, keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self) ctx:continue_with("lynessa_cast_next") end

function card.lynessa_cast_next(ctx, self)
    local choices = {}
    for index, record in ipairs(ctx:spell_cast_records(ctx:controller(self))) do
        if record.target_was_friendly_minion and ctx:get_data(self, "lynessa_used:" .. index) == 0 then
            choices[#choices + 1] = { index = index, card_id = record.card_id }
        end
    end
    if #choices > 0 then ctx:random_value(choices, "lynessa_cast_spell") end
end

function card.lynessa_cast_spell(ctx, self, choice)
    ctx:set_data(self, "lynessa_used:" .. choice.index, 1)
    ctx:cast_spell_if_valid(ctx:controller(self), choice.card_id, self)
    ctx:continue_with("lynessa_cast_next")
end

return card
