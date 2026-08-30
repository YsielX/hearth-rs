local CATEGORY = {
    felbloom = 1,
    goldthorn = 2,
    heart = 3,
    icecap = 4,
    ichor = 5,
    kingsblood = 6,
    shadow_oil = 7,
    stonescale = 8,
    netherbloom = 9,
    mystic_wool = 10,
}

local ingredient_category = {
    CFM_621t4 = CATEGORY.felbloom, CFM_621t18 = CATEGORY.felbloom, CFM_621t33 = CATEGORY.felbloom,
    CFM_621t6 = CATEGORY.goldthorn, CFM_621t24 = CATEGORY.goldthorn, CFM_621t32 = CATEGORY.goldthorn,
    CFM_621t2 = CATEGORY.heart, CFM_621t16 = CATEGORY.heart, CFM_621t25 = CATEGORY.heart,
    CFM_621t5 = CATEGORY.icecap, CFM_621t19 = CATEGORY.icecap, CFM_621t27 = CATEGORY.icecap,
    CFM_621t37 = CATEGORY.ichor, CFM_621t38 = CATEGORY.ichor, CFM_621t39 = CATEGORY.ichor,
    CFM_621t8 = CATEGORY.kingsblood, CFM_621t22 = CATEGORY.kingsblood, CFM_621t30 = CATEGORY.kingsblood,
    CFM_621t9 = CATEGORY.shadow_oil, CFM_621t23 = CATEGORY.shadow_oil, CFM_621t31 = CATEGORY.shadow_oil,
    CFM_621t3 = CATEGORY.stonescale, CFM_621t17 = CATEGORY.stonescale, CFM_621t26 = CATEGORY.stonescale,
    CFM_621t10 = CATEGORY.netherbloom, CFM_621t20 = CATEGORY.netherbloom, CFM_621t28 = CATEGORY.netherbloom,
    CFM_621t21 = CATEGORY.mystic_wool, CFM_621t29 = CATEGORY.mystic_wool,
}

local ingredients = {
    [1] = { "CFM_621t4", "CFM_621t6", "CFM_621t2", "CFM_621t5", "CFM_621t37", "CFM_621t8", "CFM_621t9", "CFM_621t3", "CFM_621t10" },
    [5] = { "CFM_621t18", "CFM_621t24", "CFM_621t16", "CFM_621t19", "CFM_621t38", "CFM_621t22", "CFM_621t23", "CFM_621t17", "CFM_621t20", "CFM_621t21" },
    [10] = { "CFM_621t33", "CFM_621t32", "CFM_621t25", "CFM_621t27", "CFM_621t39", "CFM_621t30", "CFM_621t31", "CFM_621t26", "CFM_621t28", "CFM_621t29" },
}

local potion_ids = { [1] = "CFM_621t", [5] = "CFM_621t14", [10] = "CFM_621t15" }
local tier_index = { [1] = 1, [5] = 2, [10] = 3 }

local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == wanted or tag == "all" then return true end
    end
    return false
end

local function class_eligible(definition, own_class)
    if definition.class == "neutral" or definition.class == own_class then return true end
    for _, class in ipairs(definition.classes or {}) do
        if class == own_class then return true end
    end
    return false
end

local function potion_targets(ctx, self)
    if ctx:get_data(self, "kazakus_first") == CATEGORY.heart
        or ctx:get_data(self, "kazakus_second") == CATEGORY.heart then
        return ctx:characters()
    end
    return {}
end

local function potion_on_play(ctx, self, target)
    ctx:set_data(self, "kazakus_target", target or 0)
    ctx:set_data(self, "kazakus_stage", 1)
    ctx:continue_with("kazakus_next_effect")
end

local function continue_potion(ctx) ctx:continue_with("kazakus_next_effect") end

local function begin_freezes(ctx, self, count)
    ctx:set_data(self, "kazakus_sub_remaining", count)
    ctx:continue_with("kazakus_choose_freeze")
end

local function begin_ichor(ctx, self, count)
    ctx:set_data(self, "kazakus_sub_remaining", count)
    ctx:continue_with("kazakus_choose_ichor")
end

local function begin_demons(ctx, self, count)
    ctx:set_data(self, "kazakus_sub_remaining", count)
    ctx:continue_with("kazakus_choose_demon")
end

local function apply_ingredient(ctx, self, category)
    local cost = ctx:card_definition(ctx:entity(self).card_id).cost
    local tier = tier_index[cost]
    local player = ctx:controller(self)

    if category == CATEGORY.felbloom then
        local damage = ({ 2, 4, 6 })[tier]
        local minions = ctx:minions()
        if #minions > 0 then cardlib.effects.damage_all(ctx, minions, damage) end
        continue_potion(ctx)
    elseif category == CATEGORY.goldthorn then
        local health = ({ 2, 4, 6 })[tier]
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if not is_dormant(ctx, minion) then cardlib.effects.buff(ctx, minion, 0, health) end
        end
        continue_potion(ctx)
    elseif category == CATEGORY.heart then
        local target = ctx:get_data(self, "kazakus_target")
        if target ~= 0 then
            local zone = ctx:entity(target).zone
            if zone == "hero" or zone == "board" then
                cardlib.effects.damage(ctx, target, ({ 3, 5, 8 })[tier])
            end
        end
        continue_potion(ctx)
    elseif category == CATEGORY.icecap then
        begin_freezes(ctx, self, ({ 1, 2, 3 })[tier])
    elseif category == CATEGORY.ichor then
        begin_ichor(ctx, self, ({ 1, 2, 3 })[tier])
    elseif category == CATEGORY.kingsblood then
        ctx:draw(player, ({ 1, 2, 3 })[tier])
        continue_potion(ctx)
    elseif category == CATEGORY.shadow_oil then
        begin_demons(ctx, self, ({ 1, 2, 3 })[tier])
    elseif category == CATEGORY.stonescale then
        ctx:gain_armor(player, ({ 4, 7, 10 })[tier])
        continue_potion(ctx)
    elseif category == CATEGORY.netherbloom then
        local demon = ({ "CFM_621_m4", "CFM_621_m2", "CFM_621_m3" })[tier]
        ctx:summon(player, demon)
        continue_potion(ctx)
    elseif category == CATEGORY.mystic_wool then
        if cost == 10 then
            local targets = {}
            for _, minion in ipairs(ctx:minions()) do
                if not is_dormant(ctx, minion) then targets[#targets + 1] = minion end
            end
            cardlib.effects.transform_all(ctx, targets, "CFM_621_m5")
            continue_potion(ctx)
        else
            local candidates = {}
            for _, minion in ipairs(ctx:enemy_minions(self)) do
                if not is_dormant(ctx, minion) then candidates[#candidates + 1] = minion end
            end
            if #candidates > 0 then
                ctx:random_entity(candidates, "kazakus_transform_sheep")
            else
                continue_potion(ctx)
            end
        end
    else
        continue_potion(ctx)
    end
end

local function kazakus_next_effect(ctx, self)
    local stage = ctx:get_data(self, "kazakus_stage")
    if stage > 2 then return end
    local category = stage == 1 and ctx:get_data(self, "kazakus_first")
        or ctx:get_data(self, "kazakus_second")
    ctx:set_data(self, "kazakus_stage", stage + 1)
    apply_ingredient(ctx, self, category)
end

local function kazakus_choose_freeze(ctx, self)
    if ctx:get_data(self, "kazakus_sub_remaining") <= 0 then return continue_potion(ctx) end
    local candidates = {}
    for _, minion in ipairs(ctx:enemy_minions(self)) do
        if not is_dormant(ctx, minion) and ctx:get_data(self, "kazakus_frozen:" .. minion) == 0 then
            candidates[#candidates + 1] = minion
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "kazakus_freeze") else continue_potion(ctx) end
end

local function kazakus_freeze(ctx, self, target)
    ctx:set_data(self, "kazakus_frozen:" .. target, 1)
    ctx:set_data(self, "kazakus_sub_remaining", ctx:get_data(self, "kazakus_sub_remaining") - 1)
    ctx:freeze(target)
    ctx:continue_with("kazakus_choose_freeze")
end

local function kazakus_choose_ichor(ctx, self)
    local player = ctx:controller(self)
    if ctx:get_data(self, "kazakus_sub_remaining") <= 0 or #ctx:board(player) >= 7 then
        return continue_potion(ctx)
    end
    local candidates = {}
    local seen = {}
    for _, card_id in ipairs(ctx:minions_died(player)) do
        seen[card_id] = (seen[card_id] or 0) + 1
        local used = ctx:get_data(self, "kazakus_raised:" .. card_id)
        if seen[card_id] > used then candidates[#candidates + 1] = card_id end
    end
    if #candidates > 0 then ctx:random_value(candidates, "kazakus_raise") else continue_potion(ctx) end
end

local function kazakus_raise(ctx, self, card_id)
    ctx:set_data(self, "kazakus_raised:" .. card_id, ctx:get_data(self, "kazakus_raised:" .. card_id) + 1)
    ctx:set_data(self, "kazakus_sub_remaining", ctx:get_data(self, "kazakus_sub_remaining") - 1)
    ctx:summon(ctx:controller(self), card_id)
    ctx:continue_with("kazakus_choose_ichor")
end

local function kazakus_choose_demon(ctx, self)
    if ctx:get_data(self, "kazakus_sub_remaining") <= 0 then return continue_potion(ctx) end
    local player = ctx:controller(self)
    local own_class = ctx:player(player).class
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and has_tag(definition, "demon")
            and class_eligible(definition, own_class) then
            candidates[#candidates + 1] = card_id
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "kazakus_receive_demon") else continue_potion(ctx) end
end

local function kazakus_receive_demon(ctx, self, card_id)
    ctx:set_data(self, "kazakus_sub_remaining", ctx:get_data(self, "kazakus_sub_remaining") - 1)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
    ctx:continue_with("kazakus_choose_demon")
end

local function kazakus_transform_sheep(ctx, self, target)
    cardlib.effects.transform(ctx, target, "CFM_621_m5")
    continue_potion(ctx)
end

local potion_hooks = {
    targets = potion_targets,
    on_play = potion_on_play,
    kazakus_next_effect = kazakus_next_effect,
    kazakus_choose_freeze = kazakus_choose_freeze,
    kazakus_freeze = kazakus_freeze,
    kazakus_choose_ichor = kazakus_choose_ichor,
    kazakus_raise = kazakus_raise,
    kazakus_choose_demon = kazakus_choose_demon,
    kazakus_receive_demon = kazakus_receive_demon,
    kazakus_transform_sheep = kazakus_transform_sheep,
}

local card = {
    api_version = 1,
    id = "CFM_621",
    name = "Kazakus",
    text = "[x]<b>Battlecry:</b> If your deck\nhas no duplicates,\n create a custom spell.",
    set = "GANGS",
    type = "minion",
    classes = { "mage", "priest", "warlock" },
    rarity = "legendary",
    cost = 4,
    attack = 3,
    health = 3,
    keywords = { "battlecry" },
}

local function no_duplicates(ctx, player)
    local seen = {}
    for _, entity in ipairs(ctx:deck(player)) do
        local id = ctx:entity(entity).card_id
        if seen[id] then return false end
        seen[id] = true
    end
    return true
end

function card.on_battlecry(ctx, self)
    if not no_duplicates(ctx, ctx:controller(self)) then return end
    ctx:choose_options(ctx:controller(self), "Choose a potion Cost", {
        { label = "1-Cost", value = 1 },
        { label = "5-Cost", value = 5 },
        { label = "10-Cost", value = 10 },
    }, "kazakus_cost_chosen")
end

function card.kazakus_cost_chosen(ctx, self, cost)
    ctx:set_data(self, "kazakus_cost", cost)
    ctx:discover_cards(ctx:controller(self), "Choose the first ingredient", ingredients[cost], 3, "kazakus_first_chosen")
end

function card.kazakus_first_chosen(ctx, self, card_id)
    local first = ingredient_category[card_id]
    ctx:set_data(self, "kazakus_first", first)
    local pool = {}
    for _, candidate in ipairs(ingredients[ctx:get_data(self, "kazakus_cost")]) do
        if ingredient_category[candidate] ~= first then pool[#pool + 1] = candidate end
    end
    ctx:discover_cards(ctx:controller(self), "Choose the second ingredient", pool, 3, "kazakus_second_chosen")
end

function card.kazakus_second_chosen(ctx, self, card_id)
    local player = ctx:controller(self)
    local cost = ctx:get_data(self, "kazakus_cost")
    ctx:set_data(self, "kazakus_second", ingredient_category[card_id])
    ctx:set_data(self, "kazakus_waiting_potion", 1)
    cardlib.effects.give_card(ctx, player, potion_ids[cost])
end

card.triggers = {{
    event = "card_created",
    timing = "after",
    active_zones = { "board", "graveyard" },
    condition = function(ctx, self, event)
        return event.source == self and ctx:get_data(self, "kazakus_waiting_potion") == 1
    end,
    effect = function(ctx, self, event)
        ctx:set_data(self, "kazakus_waiting_potion", 0)
        ctx:set_data(event.entity, "kazakus_first", ctx:get_data(self, "kazakus_first"))
        ctx:set_data(event.entity, "kazakus_second", ctx:get_data(self, "kazakus_second"))
    end,
}}

card.tokens = {
    { id = "CFM_621t11", name = "Lesser Potion", text = "Create a 1-Cost spell.", cost = 1, type = "spell" },
    { id = "CFM_621t12", name = "Greater Potion", text = "Create a 5-Cost spell.", cost = 5, type = "spell" },
    { id = "CFM_621t13", name = "Superior Potion", text = "Create a 10-Cost spell.", cost = 10, type = "spell" },
    { id = "CFM_621t4", name = "Felbloom", text = "Deal $2 damage to all minions.", cost = 1, type = "spell" },
    { id = "CFM_621t6", name = "Goldthorn", text = "Give your minions +2 Health.", cost = 1, type = "spell" },
    { id = "CFM_621t2", name = "Heart of Fire", text = "Deal $3 damage.", cost = 1, type = "spell" },
    { id = "CFM_621t5", name = "Icecap", text = "<b>Freeze</b> a random enemy minion.", cost = 1, type = "spell" },
    { id = "CFM_621t37", name = "Ichor of Undeath", text = "Summon a friendly minion that died this game.", cost = 1, type = "spell" },
    { id = "CFM_621t8", name = "Kingsblood", text = "Draw a card.", cost = 1, type = "spell" },
    { id = "CFM_621t9", name = "Shadow Oil", text = "Add a random Demon to your hand.", cost = 1, type = "spell" },
    { id = "CFM_621t3", name = "Stonescale Oil", text = "Gain 4 Armor.", cost = 1, type = "spell" },
    { id = "CFM_621t10", name = "Netherbloom", text = "Summon a 2/2 Demon.", cost = 1, type = "spell" },
    { id = "CFM_621t21", name = "Mystic Wool", text = "Transform a random enemy minion into a 1/1 Sheep.", cost = 5, type = "spell" },
    { id = "CFM_621_m4", name = "Kabal Demon", text = "", cost = 2, type = "minion", attack = 2, health = 2, tags = { "demon" } },
    { id = "CFM_621_m5", name = "Sheep", text = "", cost = 1, type = "minion", attack = 1, health = 1, tags = { "beast" } },
    { id = "CFM_621t", name = "Kazakus Potion", text = "{0}\n{1}", cost = 1, type = "spell" },
    { id = "CFM_621t14", name = "Kazakus Potion", text = "{0}\n{1}", cost = 5, type = "spell" },
    { id = "CFM_621t15", name = "Kazakus Potion", text = "{0}\n{1}", cost = 10, type = "spell" },
    { id = "CFM_621t16", name = "Heart of Fire", text = "Deal $5 damage.", cost = 5, type = "spell" },
    { id = "CFM_621t17", name = "Stonescale Oil", text = "Gain 7 Armor.", cost = 5, type = "spell" },
    { id = "CFM_621t18", name = "Felbloom", text = "Deal $4 damage to all minions.", cost = 5, type = "spell" },
    { id = "CFM_621t19", name = "Icecap", text = "<b>Freeze</b> 2 random enemy minions.", cost = 5, type = "spell" },
    { id = "CFM_621t20", name = "Netherbloom", text = "Summon a 5/5 Demon.", cost = 5, type = "spell" },
    { id = "CFM_621t22", name = "Kingsblood", text = "Draw 2 cards.", cost = 5, type = "spell" },
    { id = "CFM_621t23", name = "Shadow Oil", text = "Add 2 random Demons to your hand.", cost = 5, type = "spell" },
    { id = "CFM_621t24", name = "Goldthorn", text = "Give your minions +4 Health.", cost = 5, type = "spell" },
    { id = "CFM_621t25", name = "Heart of Fire", text = "Deal $8 damage.", cost = 10, type = "spell" },
    { id = "CFM_621t26", name = "Stonescale Oil", text = "Gain 10 Armor.", cost = 10, type = "spell" },
    { id = "CFM_621t27", name = "Icecap", text = "<b>Freeze</b> 3 random enemy minions.", cost = 10, type = "spell" },
    { id = "CFM_621t28", name = "Netherbloom", text = "Summon an 8/8 Demon.", cost = 10, type = "spell" },
    { id = "CFM_621t29", name = "Mystic Wool", text = "Transform all minions into 1/1 Sheep.", cost = 10, type = "spell" },
    { id = "CFM_621t30", name = "Kingsblood", text = "Draw 3 cards.", cost = 10, type = "spell" },
    { id = "CFM_621t31", name = "Shadow Oil", text = "Add 3 random Demons to your hand.", cost = 10, type = "spell" },
    { id = "CFM_621t32", name = "Goldthorn", text = "Give your minions +6 Health.", cost = 10, type = "spell" },
    { id = "CFM_621t33", name = "Felbloom", text = "Deal $6 damage to all minions.", cost = 10, type = "spell" },
    { id = "CFM_621_m2", name = "Kabal Demon", text = "", cost = 5, type = "minion", attack = 5, health = 5, tags = { "demon" } },
    { id = "CFM_621_m3", name = "Kabal Demon", text = "", cost = 8, type = "minion", attack = 8, health = 8, tags = { "demon" } },
    { id = "CFM_621t38", name = "Ichor of Undeath", text = "Summon 2 friendly minions that died this game.", cost = 5, type = "spell" },
    { id = "CFM_621t39", name = "Ichor of Undeath", text = "Summon 3 friendly minions that died this game.", cost = 10, type = "spell" },
}

for _, token in ipairs(card.tokens) do
    token.set = "GANGS"
    token.class = "neutral"
    token.collectible = false
    if token.id == "CFM_621t" or token.id == "CFM_621t14" or token.id == "CFM_621t15" then
        token.target_mode = "required_if_available"
        for hook, implementation in pairs(potion_hooks) do token[hook] = implementation end
    end
end

return card
