local powers = {
    warrior = "HERO_01bp", shaman = "HERO_02bp", rogue = "HERO_03bp",
    paladin = "HERO_04bp", hunter = "HERO_05bp", druid = "HERO_06bp",
    mage = "HERO_08bp", priest = "HERO_09bp", demon_hunter = "HERO_10bp",
    death_knight = "HERO_11bp",
}

local function replacements(ctx, class, rarity)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.class == class and definition.rarity == rarity then
            result[#result + 1] = card_id
        end
    end
    return result
end

local card = {
    api_version = 1, id = "OG_118", name = "Renounce Darkness",
    text = "Replace your Hero Power and Warlock cards with another class's. The cards cost (1) less.",
    set = "OG", type = "spell", class = "warlock", rarity = "epic", cost = 2,
    spell_school = "holy",
}
function card.on_play(ctx, self)
    local classes = {}
    for class, _ in pairs(powers) do classes[#classes + 1] = class end
    table.sort(classes)
    ctx:random_value(classes, "choose_new_class")
end
function card.choose_new_class(ctx, self, class)
    ctx:set_data(self, "renounce_phase", 1)
    ctx:set_data(self, "renounce_index", 1)
    ctx:set_data(self, "renounce_class_index", 0)
    for index, candidate in ipairs({ "death_knight", "demon_hunter", "druid", "hunter", "mage", "paladin", "priest", "rogue", "shaman", "warrior" }) do
        if candidate == class then ctx:set_data(self, "renounce_class_index", index) break end
    end
    ctx:set_player_class(ctx:controller(self), class)
    ctx:replace_hero_power(ctx:controller(self), powers[class])
    ctx:continue_with("replace_next_warlock_card")
end
function card.replace_next_warlock_card(ctx, self)
    local player = ctx:controller(self)
    local phase = ctx:get_data(self, "renounce_phase") or 1
    local index = ctx:get_data(self, "renounce_index") or 1
    local zone = phase == 1 and ctx:hand(player) or ctx:deck(player)
    while index <= #zone and ctx:card_definition(ctx:entity(zone[index]).card_id).class ~= "warlock" do
        index = index + 1
    end
    if index > #zone then
        if phase == 1 then
            ctx:set_data(self, "renounce_phase", 2)
            ctx:set_data(self, "renounce_index", 1)
            ctx:continue_with("replace_next_warlock_card")
        end
        return
    end
    local target = zone[index]
    local classes = { "death_knight", "demon_hunter", "druid", "hunter", "mage", "paladin", "priest", "rogue", "shaman", "warrior" }
    local class = classes[ctx:get_data(self, "renounce_class_index")]
    local rarity = ctx:card_definition(ctx:entity(target).card_id).rarity
    local pool = replacements(ctx, class, rarity)
    ctx:set_data(self, "renounce_target", target)
    ctx:set_data(self, "renounce_index", index)
    if #pool > 0 then ctx:random_value(pool, "finish_renouncing_card")
    else
        ctx:set_data(self, "renounce_index", index + 1)
        ctx:continue_with("replace_next_warlock_card")
    end
end
function card.finish_renouncing_card(ctx, self, card_id)
    local target = ctx:get_data(self, "renounce_target")
    cardlib.effects.transform(ctx, target, card_id)
    cardlib.effects.modify(ctx, target, { stat = "cost", operation = "add", value = -1 })
    ctx:set_data(self, "renounce_index", (ctx:get_data(self, "renounce_index") or 1) + 1)
    ctx:continue_with("replace_next_warlock_card")
end
return card
