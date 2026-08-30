local effects = {
    api_version = 1,
    module_type = "library",
    id = "effects",
}

local function copy_table(value)
    local copy = {}
    if value ~= nil then
        for key, entry in pairs(value) do copy[key] = entry end
    end
    return copy
end

local function equal_hits(targets, amount)
    local hits = {}
    for _, target in ipairs(targets) do
        hits[#hits + 1] = { target, amount }
    end
    return hits
end

function effects.damage(ctx, target, amount, options)
    ctx:damage_batch({ { target, amount } }, options)
end

function effects.damage_all(ctx, targets, amount, options)
    ctx:damage_batch(equal_hits(targets, amount), options)
end

function effects.damage_batch(ctx, hits, options)
    ctx:damage_batch(hits, options)
end

function effects.damage_ignoring_spell_damage(ctx, target, amount)
    ctx:damage_batch({ { target, amount } }, { apply_spell_damage = false })
end

function effects.damage_batch_ignoring_spell_damage(ctx, hits)
    ctx:damage_batch(hits, { apply_spell_damage = false })
end

function effects.damage_from(ctx, source, target, amount)
    ctx:damage_batch({ { target, amount } }, {
        source = source,
        apply_spell_damage = false,
    })
end

function effects.damage_batch_from(ctx, source, hits)
    ctx:damage_batch(hits, {
        source = source,
        apply_spell_damage = false,
    })
end

function effects.heal(ctx, target, amount)
    ctx:heal_batch({ { target, amount } })
end

function effects.heal_all(ctx, targets, amount)
    ctx:heal_batch(equal_hits(targets, amount))
end

function effects.heal_batch(ctx, hits)
    ctx:heal_batch(hits)
end

function effects.destroy(ctx, target)
    ctx:destroy_batch({ target })
end

function effects.destroy_all(ctx, targets)
    ctx:destroy_batch(targets)
end

function effects.destroy_batch(ctx, targets)
    ctx:destroy_batch(targets)
end

function effects.transform(ctx, target, card_id)
    ctx:transform_batch({ { target, card_id } })
end

function effects.transform_all(ctx, targets, card_id)
    local transforms = {}
    for _, target in ipairs(targets) do
        transforms[#transforms + 1] = { target, card_id }
    end
    ctx:transform_batch(transforms)
end

function effects.transform_batch(ctx, transforms)
    ctx:transform_batch(transforms)
end

function effects.transform_preserving_scripts(ctx, target, card_id)
    ctx:transform_batch({ { target, card_id } }, {
        preserve_attached_scripts = true,
    })
end

function effects.transform_into_copy_with_stats(ctx, target, template, attack, health)
    ctx:transform_into_copy(target, template, {
        final_stats = { attack = attack, health = health },
    })
end

function effects.summon_at(ctx, player, card_id, position)
    ctx:summon(player, card_id, { position = position })
end

function effects.summon_with_stats(ctx, player, card_id, attack, health, keywords)
    ctx:summon(player, card_id, {
        final_stats = { attack = attack, health = health },
        keywords = keywords,
    })
end

function effects.summon_with_base_stats(ctx, player, card_id, attack, health, keywords)
    ctx:summon(player, card_id, {
        base_stats = { attack = attack, health = health },
        keywords = keywords,
    })
end

function effects.summon_existing_at(ctx, player, target, position)
    ctx:summon_existing(player, target, { position = position })
end

function effects.recruit_at(ctx, player, target, position)
    ctx:recruit(player, target, { position = position })
end

function effects.move_to_hand(ctx, player, target)
    ctx:move(target, "hand", { player = player })
end

function effects.shuffle_entity_into_deck(ctx, player, target)
    ctx:move(target, "deck_random", { player = player })
end

function effects.give_copy_with_stats(ctx, player, target, attack, health, cost)
    ctx:give_copy(player, target, {
        final_stats = { attack = attack, health = health },
        cost = cost,
    })
end

function effects.give_base_copy(ctx, player, target)
    ctx:give_copy(player, target, { state = "definition" })
end

function effects.give_base_copy_with_stats(ctx, player, target, attack, health, cost)
    ctx:give_copy(player, target, {
        state = "definition",
        final_stats = { attack = attack, health = health },
        cost = cost,
    })
end

function effects.give_card(ctx, player, card_id)
    ctx:create_card(player, card_id)
end

function effects.give_card_at(ctx, player, card_id, position)
    ctx:create_card(player, card_id, { destination = "hand", position = position })
end

function effects.shuffle_card_into_deck(ctx, player, card_id)
    ctx:create_card(player, card_id, { destination = "deck_random" })
end

function effects.buff(ctx, target, attack, health)
    ctx:buff(target, { attack = attack, health = health })
end

function effects.buff_until_end_of_turn(ctx, target, attack, health)
    ctx:buff(target, {
        attack = attack,
        health = health,
        duration = "end_of_turn",
    })
end

function effects.grant_keyword(ctx, target, keyword)
    ctx:buff(target, { keywords = { keyword } })
end

function effects.grant_keyword_until_end_of_turn(ctx, target, keyword)
    ctx:buff(target, {
        keywords = { keyword },
        duration = "end_of_turn",
    })
end

function effects.summon_copy_at(ctx, player, target, position)
    ctx:summon_copy(player, target, { position = position })
end

function effects.summon_copy_with_stats(ctx, player, target, attack, health)
    ctx:summon_copy(player, target, {
        final_stats = { attack = attack, health = health },
    })
end

function effects.summon_fresh_copy(ctx, target, position, health, without_keywords)
    ctx:summon_fresh_copy(target, {
        position = position,
        remaining_health = health,
        without_keywords = without_keywords,
    })
end

function effects.summon_fresh_copy_with_stats(ctx, target, position, attack, health, without_keywords)
    ctx:summon_fresh_copy(target, {
        position = position,
        final_stats = { attack = attack, health = health },
        without_keywords = without_keywords,
    })
end

local function modification(target, spec)
    local entry = copy_table(spec)
    entry.target = target
    if spec.stat ~= nil then
        entry.modifiers = { {
            stat = spec.stat,
            operation = spec.operation,
            value = spec.value,
        } }
    end
    return entry
end

function effects.modify(ctx, target, spec)
    ctx:modify_batch({ modification(target, spec) })
end

function effects.modify_all(ctx, targets, spec)
    local modifications = {}
    for _, target in ipairs(targets) do
        modifications[#modifications + 1] = modification(target, spec)
    end
    ctx:modify_batch(modifications)
end

function effects.modify_batch(ctx, modifications)
    ctx:modify_batch(modifications)
end

function effects.set_event_amount(ctx, event, amount)
    ctx:modify_event_amount(event, {
        operation = "set",
        value = amount,
    })
end

function effects.add_event_amount(ctx, event, amount)
    ctx:modify_event_amount(event, {
        operation = "add",
        value = amount,
    })
end

function effects.multiply_event_amount(ctx, event, factor)
    ctx:modify_event_amount(event, {
        operation = "multiply",
        value = factor,
    })
end

return effects
